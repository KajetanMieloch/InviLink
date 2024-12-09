from . import views
from django.urls import path

urlpatterns = [
    path('phantom_test/', views.phantom_test, name='phantom_test'),
    path('blockchain-data/', views.get_blockchain_data, name='blockchain_data'),
    # path('phantom_callback/', views.phantom_callback, name='phantom_callback'),
    # path('phantom_success/', views.phantom_success, name='phantom_success'),
]