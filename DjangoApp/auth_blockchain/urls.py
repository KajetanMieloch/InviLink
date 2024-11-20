from django.urls import path
from . import views

urlpatterns = [
    path('phantom/login/', views.phantom_mobile_login, name='phantom_mobile_login'),
    path('phantom/callback/', views.phantom_callback, name='phantom_callback'),
]
